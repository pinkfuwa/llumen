use tower_http::{CompressionLevel, compression::CompressionLayer as TowerCompressionLayer};

#[derive(Debug, Default, Clone, Copy)]
pub struct ZstdCompressionLayer;

impl<S> tower::Layer<S> for ZstdCompressionLayer {
    type Service = tower_http::compression::Compression<S>;

    fn layer(&self, inner: S) -> Self::Service {
        // only zstd, to make better server binary size
        TowerCompressionLayer::new()
            .no_deflate()
            .no_br()
            .no_gzip()
            .zstd(true)
            .quality(CompressionLevel::Precise(1))
            .layer(inner)
    }
}
