use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        description = "API for DeepDecipher. See [web page](/) and [GitHub](https://github.com/apartresearch/deepdecipher) for more information."
    ),
    paths(
        super::response::api_index,
        super::response::model,
        super::response::layer,
        super::response::neuron,
        super::response::all_model,
        super::response::all_layer,
        super::response::all_neuron,
        super::response::api_doc,
    )
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
    }

    doc
}
