use crate::ConfigMap;

/// A factory to instantiate [`DynSynTripleSerializer`](super::sync::DynSynTripleSerializer).
#[derive(Debug, Default, Clone)]
pub struct DynSynTripleSerializerFactory {
    serializer_config_map: ConfigMap,
}

impl DynSynTripleSerializerFactory {
    /// Instantiate a factory. It takes a
    /// `serializer_config_map`, an optional [`ConfigMap`], which
    /// can be populated with configuration structures
    /// corresponding to supported syntaxes.
    #[inline]
    pub fn new(serializer_config_map: Option<ConfigMap>) -> Self {
        Self {
            serializer_config_map: serializer_config_map.unwrap_or_default(),
        }
    }

    /// Get serializer config of given type.
    #[inline]
    pub fn get_config<T: Clone + Default + Send + Sync + 'static>(&self) -> T {
        self.serializer_config_map
            .get::<T>()
            .cloned()
            .unwrap_or_default()
    }
}
