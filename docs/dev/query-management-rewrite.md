# Query Management Rewrite - Svelte 5 Runes Migration

This document describes the major rewrite of the query/state management system from Svelte 4 stores to Svelte 5 runes.

## Overview

The query management system has been completely rewritten to leverage Svelte 5's runes (`$state`, `$derived`, `$effect`) instead of Svelte stores. This provides better reactivity, simpler code, and improved performance.

## Key Changes

### 1. Module-Level State

**Before (Svelte 4 with stores):**
```typescript
// Queries returned stores
const { data: models } = useModels();
// Usage: $models
```

**After (Svelte 5 with runes):**
```typescript
// State declared at module level
let models = $state<ModelListResp | undefined>(undefined);

// Query effect initializes/updates state
export function useModelsQueryEffect() {
  createQueryEffect({
    path: 'model/list',
    body: {},
    updateData: (data) => { models = data; }
  });
}

// Getter function to read state
export function getModels(): ModelListResp | undefined {
  return models;
}
```

### 2. No More Keys or Global Cache

**Before:**
- Queries were identified by keys (e.g., `key: ['models']`)
- Data stored in global cache
- Used `setContext`/`getContext` for sharing

**After:**
- No keys needed - direct module-level state
- No global cache
- Use getter functions instead of context

### 3. Query Effects Must Be Called During Component Initialization

**Before:**
```svelte
<script>
  const { data: models } = useModels();
</script>
```

**After:**
```svelte
<script>
  // Call effect during initialization
  useModelsQueryEffect();
  
  // Read data with getter
  const models = $derived(getModels());
</script>
```

### 4. Mutation API Changes

**Before:**
```typescript
const { mutate, isPending, isError } = CreateMutation({...});
// isPending and isError were stores: $isPending, $isError
```

**After:**
```typescript
const { mutate, isPending, isError } = createMutation({...});
// isPending and isError are functions: isPending(), isError()
```

### 5. Flattened Infinite Query Structure

**Before:**
- Complex `Pages` class with methods
- `Page` objects with `data` as Writable stores
- Used `SetInfiniteQueryData`, `UpdateInfiniteQueryDataById`, etc.

**After:**
- Simple `PageState<D>` interface with plain data arrays
- Pure functions for data manipulation
- Module-level state array

**Example:**
```typescript
// Module-level state
let roomPages = $state<PageState<ChatPaginateRespList>[]>([]);

// Query effect
export function useRoomsQueryEffect() {
  createInfiniteQueryEffect({
    fetcher: new ChatFetcher(),
    updatePages: (updater) => { roomPages = updater(roomPages); },
    getPages: () => roomPages
  });
}

// Pure functions for updates
roomPages = insertInfiniteQueryData(roomPages, newItem);
roomPages = updateInfiniteQueryDataById(roomPages, id, updater);
```

## Migration Guide

### For Query APIs

1. **Declare module-level state:**
   ```typescript
   let myData = $state<MyType | undefined>(undefined);
   ```

2. **Create query effect function:**
   ```typescript
   export function useMyDataQueryEffect() {
     createQueryEffect({
       path: 'my/endpoint',
       body: {},
       updateData: (data) => { myData = data; }
     });
   }
   ```

3. **Create getter function:**
   ```typescript
   export function getMyData(): MyType | undefined {
     return myData;
   }
   ```

4. **Optional: Create setter function:**
   ```typescript
   export function setMyData(data: MyType | undefined) {
     myData = data;
   }
   ```

### For Components

1. **Replace `useQuery()` calls with `useQueryEffect()`:**
   ```typescript
   // Before
   const { data: models } = useModels();
   
   // After
   useModelsQueryEffect();
   const models = $derived(getModels());
   ```

2. **Remove `setContext`/`getContext`:**
   ```typescript
   // Before
   setContext('models', models);
   const models = getContext<Readable<ModelListResp>>('models');
   
   // After
   useModelsQueryEffect(); // in parent component
   const models = $derived(getModels()); // in any component
   ```

3. **Update mutation usage:**
   ```typescript
   // Before
   const { mutate, isPending } = CreateMutation({...});
   if ($isPending) { ... }
   
   // After
   const { mutate, isPending } = createMutation({...});
   if (isPending()) { ... }
   ```

### For Infinite Queries

1. **Declare page state:**
   ```typescript
   let pages = $state<PageState<MyItem>[]>([]);
   ```

2. **Create query effect:**
   ```typescript
   export function useMyItemsQueryEffect() {
     createInfiniteQueryEffect({
       fetcher: new MyFetcher(),
       updatePages: (updater) => { pages = updater(pages); },
       getPages: () => pages
     });
   }
   ```

3. **Use pure functions for updates:**
   ```typescript
   pages = insertInfiniteQueryData(pages, newItem);
   pages = updateInfiniteQueryDataById(pages, id, (item) => ({...item, field: value}));
   pages = removeInfiniteQueryData(pages, (item) => item.id < threshold);
   ```

4. **Update component usage:**
   ```typescript
   // Before
   const { data } = useRooms();
   {#each $data as page}
     {#each $page.data as item}
   
   // After
   useRoomsQueryEffect();
   const pages = $derived(getRoomPages());
   {#each pages as page}
     {#each page.data as item}
   ```

## Benefits

1. **Simpler Mental Model**: Direct state management without indirection of keys/cache
2. **Better Performance**: Runes are more efficient than stores
3. **Type Safety**: Better TypeScript inference with direct state
4. **Easier Testing**: Pure functions are easier to test
5. **Less Boilerplate**: No need to manage keys or cache
6. **Clearer Data Flow**: Explicit getter/setter functions

## Testing

A comprehensive test suite has been added for infinite query utilities (`frontend/src/lib/api/state/infinite.test.ts`) covering:
- Data insertion
- Updates by ID
- Continuous removal from beginning
- Edge cases and immutability

Run tests with:
```bash
cd frontend && pnpm test infinite.test.ts
```

## Files Modified

### Core State Management
- `frontend/src/lib/api/state/query.ts` - Rewritten with runes
- `frontend/src/lib/api/state/mutate.ts` - Rewritten with runes
- `frontend/src/lib/api/state/infinite.ts` - Rewritten with flattened structure
- `frontend/src/lib/api/state/index.ts` - Updated exports
- `frontend/src/lib/api/state/mock.ts` - Updated for new API

### API Modules
- `frontend/src/lib/api/model.ts` - Migrated to new pattern
- `frontend/src/lib/api/user.ts` - Migrated to new pattern
- `frontend/src/lib/api/chatroom.svelte.ts` - Migrated to new pattern
- `frontend/src/lib/api/message.svelte.ts` - Updated mutations
- `frontend/src/lib/api/auth.ts` - Updated mutations
- `frontend/src/lib/api/index.ts` - Updated exports

### Components
- `frontend/src/routes/chat/+layout.svelte` - Removed context, added query effects
- `frontend/src/routes/chat/[id]/+page.svelte` - Updated room query usage
- `frontend/src/lib/components/room/RoomPagination.svelte` - Updated infinite query
- `frontend/src/lib/components/room/Page.svelte` - Updated page state handling
- `frontend/src/lib/components/setting/ModelGrid.svelte` - Removed context
- `frontend/src/lib/components/setting/UserGrid.svelte` - Removed context, added query effect
- `frontend/src/lib/components/input/ModelSelector.svelte` - Removed context
- `frontend/src/lib/components/codemirror/Toml.svelte` - Updated for new API
- `frontend/src/lib/components/setting/tabs/account/*.svelte` - Updated mutations
- `frontend/src/lib/components/setting/tabs/admin/*.svelte` - Updated mutations
- `frontend/src/routes/login/+page.svelte` - Updated mutation usage

### Tests
- `frontend/src/lib/api/state/infinite.test.ts` - New comprehensive test suite

## Breaking Changes

All query/mutation APIs have been renamed to use camelCase and the old store-based APIs have been removed. Components must be updated to use the new patterns as described in this document.