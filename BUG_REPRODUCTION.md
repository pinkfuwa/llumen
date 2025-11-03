# Bug Reproduction: Duplicate Chunks in Streaming Messages

## Issue Description
Same chunks appear twice within the same message during SSE (Server-Sent Events) streaming in the frontend.

## Root Cause
Race condition in `frontend/src/lib/api/message.svelte.ts` in the `useSSEEffect` function (lines 132-158).

### Technical Details
The bug occurs when the page visibility changes (user switches tabs):

1. **Initial state**: SSE connection is active and streaming chunks
2. **User switches away**: Page visibility becomes 'hidden', controller is aborted (line 146)
3. **User switches back**: Page visibility becomes 'visible'
   - Old controller is aborted (if not already) at line 143
   - NEW controller is immediately created at line 144
   - NEW SSE connection starts at line 145

**The Race Condition:**
- The abort signal is passed to the fetch request, but the `for await` loop in `startSSE` (line 110) doesn't check if the signal is aborted before processing each event
- When the old connection is aborted, some SSE events may already be in the event stream buffer
- The old connection continues to process these buffered events even after abort is called
- The new connection ALSO receives these same events from the server
- Both connections process the same events, calling the same handlers, resulting in duplicate chunks being added to the message

### Code Flow Diagram
```
User switches tab back to page
  ↓
onVisibilityChange('visible') fires
  ↓
controller.abort() called (line 143)
  ↓ (but old SSE loop might still be processing events)
  ↓
new AbortController() created (line 144)
  ↓
startSSE() called with new controller (line 145)
  ↓
[OLD CONNECTION] Still processing buffered events
[NEW CONNECTION] Also receiving same events
  ↓
Both call Handlers.token() / Handlers.reasoning() etc.
  ↓
handleTokenChunk() appends content twice
  ↓
DUPLICATE CHUNKS APPEAR IN UI
```

## Steps to Reproduce

### Scenario 1: Tab Switching During Streaming
1. Start the llumen application
2. Navigate to a chat room
3. Send a message that triggers streaming response (e.g., ask "Write a long story about space exploration")
4. While the response is streaming (chunks are arriving):
   - Switch to another browser tab
   - Wait 1-2 seconds
   - Switch back to the llumen tab
5. **Expected**: Streaming continues normally without duplicates
6. **Actual**: Some chunks that arrived during the tab switch appear twice in the message

### Scenario 2: Repeated Tab Switching
1. Start streaming a response
2. Rapidly switch away and back to the tab multiple times (3-5 times)
3. **Expected**: Message displays correctly
4. **Actual**: Multiple chunks are duplicated, message becomes garbled with repeated text

### Scenario 3: With Version Change
1. Have two browser windows/tabs open to the same chat
2. Start streaming in Window A
3. While streaming, send another message from Window B (this updates the version)
4. Switch away from and back to Window A during the streaming
5. **Expected**: Window A syncs messages and displays correctly
6. **Actual**: Duplicate chunks appear due to both the version sync and the visibility change race condition

### Scenario 4: Image Upload with Tab Switch
1. Start a chat with an image attached
2. Send the message to trigger streaming
3. Switch tabs while the initial response chunks are arriving
4. Switch back
5. **Expected**: Response displays correctly
6. **Actual**: Initial chunks may be duplicated

## Visual Evidence
When the bug occurs, you'll see in the UI:
- Repeated words or sentences in the streaming text
- Duplicate reasoning blocks
- Multiple identical chunks appearing in rapid succession

## Files Affected
- `frontend/src/lib/api/message.svelte.ts` - Main file with the bug (lines 132-158)

## Fix Summary
The fix must ensure that:
1. Old SSE connection is fully terminated before starting a new one
2. Events from an aborted connection are not processed
3. Only one active SSE connection exists at any time
4. The abort signal is checked in the event processing loop
