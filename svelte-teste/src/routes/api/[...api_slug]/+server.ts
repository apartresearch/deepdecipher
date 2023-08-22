import { API_EXT, BASE_API_URL } from '$lib/base';

export async function GET({ params }: { params: { api_slug: string } }) {
    const { api_slug } = params;
    const api_url = `${BASE_API_URL}/${API_EXT}/${api_slug}`;
    return fetch(api_url)
}
