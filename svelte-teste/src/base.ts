//export const BASE_API_URL: string = "https://deepdecipher.org"
export const BASE_API_URL: string = "http://127.0.0.1:8080"
export const BASE_VIZ_API: string = "viz";
export const BASE_EXT_API: string = "api";

export async function getModelMetadata(modelName: string): Promise<Record<string, any> | string> {
    const url = `${BASE_API_URL}/${BASE_EXT_API}/${modelName}/metadata`;
    const response = await fetch(
        url
    );
    if (response.ok) {
        return (await response.json()).data;
    } else {
        return response.text();
    }
}