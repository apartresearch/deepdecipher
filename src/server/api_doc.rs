use utoipa::OpenApi;

use crate::data::{NeuronIndex, NeuroscopeModelPage};

#[derive(OpenApi)]
#[openapi(
    paths(
        super::response::api_index,
        super::response::model,
        super::response::base
    ),
    components(schemas(NeuroscopeModelPage, NeuronIndex))
)]
pub struct ApiDoc;

pub fn api_doc() -> utoipa::openapi::OpenApi {
    let mut doc = ApiDoc::openapi();
    for operation in doc
        .paths
        .paths
        .values_mut()
        .flat_map(|path| path.operations.values_mut())
    {
        operation.tags = None;
        operation.operation_id = None;
    }
    doc
}
