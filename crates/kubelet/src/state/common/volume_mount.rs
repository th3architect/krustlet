//! Kubelet is pulling container images.

use log::error;

use crate::state::prelude::*;
use crate::volume::Ref;

use super::{GenericPodState, GenericProvider, GenericProviderState};
use crate::state::common::error::Error;

/// Kubelet is pulling container images.
pub struct VolumeMount<P: GenericProvider> {
    phantom: std::marker::PhantomData<P>,
}

impl<P: GenericProvider> std::fmt::Debug for VolumeMount<P> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "VolumeMount".fmt(formatter)
    }
}

impl<P: GenericProvider> Default for VolumeMount<P> {
    fn default() -> Self {
        Self {
            phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<P: GenericProvider> State<P::ProviderState, P::PodState> for VolumeMount<P> {
    async fn next(
        self: Box<Self>,
        provider_state: SharedState<P::ProviderState>,
        pod_state: &mut P::PodState,
        pod: &Pod,
    ) -> Transition<P::ProviderState, P::PodState> {
        let (client, volume_path) = {
            let state_reader = provider_state.read().await;
            (state_reader.client(), state_reader.volume_path())
        };
        let volumes = match Ref::volumes_from_pod(&volume_path, &pod, &client).await {
            Ok(v) => v,
            Err(e) => {
                error!("{:?}", e);
                let next = Error::<P>::new(e.to_string());
                return Transition::next(self, next);
            }
        };
        pod_state.set_volumes(volumes);
        Transition::next_unchecked(self, P::RunState::default())
    }

    async fn json_status(
        &self,
        _pod_state: &mut P::PodState,
        _pod: &Pod,
    ) -> anyhow::Result<serde_json::Value> {
        make_status(Phase::Pending, "VolumeMount")
    }
}

impl<P: GenericProvider> TransitionTo<Error<P>> for VolumeMount<P> {}
