import { API_EXT, BASE_API_URL } from '$lib/base';

export async function GET({ params, url }: { params: { api_slug: string }, url: URL }) {
    const { api_slug } = params;
    const api_url = `${BASE_API_URL}/${API_EXT}/${api_slug}?${url.searchParams.toString()}`;
    return fetch(api_url)
}
