// Cap for initial message/chat page load until frontend pagination is fully
// implemented. 200 is conservative to prevent loading too many records at once.
pub const MAX_PAGINATE_LIMIT: u32 = 200;

// Default server bind address
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8001";

// SQLite busy timeout in milliseconds - how long to wait for locks to clear
pub const DB_BUSY_TIMEOUT_MS: u32 = 1000;

// Cache duration for compressed images: 259200 seconds = 3 days
pub const IMAGE_CACHE_MAX_AGE_SECS: u32 = 259200;

// Token expiration time: 7 days in seconds
pub const TOKEN_EXPIRATION_SECS: u64 = 60 * 60 * 24 * 7;

// Cache size of sqlite in byte: 32MiB
pub const DB_CACHE_SIZE: usize = 1024 * 1024 * 32;

// title generation temperature
pub const TITLE_GENERATION_TEMPERATURE: f32 = 0.2;
