import { redirect } from '@sveltejs/kit';

export async function load({ params }: { params: { model: string } }) {
    throw redirect(303, `/viz/${params.model}/all`);
}
