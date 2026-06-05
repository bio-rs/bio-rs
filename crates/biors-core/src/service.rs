//! Service contracts for local and embedded bio-rs integrations.
//!
//! `biors-core` owns deterministic request/response contracts and schema
//! metadata. HTTP listeners, authentication, rate limits, background queues,
//! object storage, and deployment policy remain outside the core crate.

#[path = "service/batch.rs"]
mod batch;
#[path = "service/hosted.rs"]
mod hosted;
#[path = "service/http.rs"]
mod http;
#[path = "service/interface.rs"]
mod interface;

pub use batch::{
    validate_service_batch_sequence_request, ServiceBatchSequenceInput,
    ServiceBatchSequenceItemReport, ServiceBatchSequenceSummary,
    ServiceBatchSequenceValidateOutput, ServiceBatchSequenceValidateRequest,
    ServiceBatchValidationError, ServiceSequenceKindSelection,
    SERVICE_BATCH_SEQUENCE_VALIDATE_SCHEMA_VERSION,
};
pub use hosted::{
    current_hosted_workflow_boundary, hosted_workflow_boundary, HostedCommercialPolicy,
    HostedExecutionPolicy, HostedResponsibilitySet, HostedWebProductPolicy, HostedWorkflowBoundary,
    HostedWorkspaceConcept, HOSTED_WORKFLOW_BOUNDARY_SCHEMA_VERSION,
};
pub use http::{
    service_health_document, service_openapi_document, ServiceEndpointStatus,
    ServiceHealthDocument, SERVICE_HEALTH_SCHEMA_VERSION,
};
pub use interface::{
    current_service_interface_document, local_service_routes, service_interface_document,
    service_routes, OpenApiDirection, RuntimeServiceSeparation, ServiceInterfaceDocument,
    ServiceRoute, SERVICE_INTERFACE_SCHEMA_VERSION,
};
