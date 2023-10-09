//! I provide an implementation of [`AdminPod`] that manages 
//! member pods based on a configured template.
//! 

use std::sync::Arc;

use dashmap::DashSet;

use crate::pod::impl_::BasicPod;

use self::pod_template::PodTemplate;

pub mod pod_template;


/// An implementation of [`AdminPod`] that manages 
/// member pods based on a configured template.
#[derive(Debug, Clone)]
pub struct TemplateDrivenAdminPod<Inner, MTemplate: PodTemplate> {
    /// Inner pod.
    inner: Inner,
    
    /// Member template.
    member_template: MTemplate,

    /// Provisioned member keys.
    provisioned_member_keys: Arc<DashSet<MTemplate::PodKey>>,

    // provisioned_pod_cache: 
}
