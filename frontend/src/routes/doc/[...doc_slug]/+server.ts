import { BASE_API_URL } from '$lib/base';

export async function GET({ params }: { params: { doc_slug: string } }) {
    const { doc_slug } = params;
    const doc_url = `${BASE_API_URL}/doc/${doc_slug}`;
    return fetch(doc_url)
}
