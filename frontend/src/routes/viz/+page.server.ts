import { getModels } from '$lib/modelMetadata';
import { error } from '@sveltejs/kit';

export async function load({ params }: { params: { model: string, service: string } }) {
    const models = await getModels();
    if (typeof models === "string")
        error(500, models);
    return { models };
}
