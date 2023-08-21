<script lang="ts">
	import SearchResults from './SearchResults.svelte';
	import { search } from '$lib/n2g_search';
	import { VIZ_EXT } from '$lib/base';
	import type { Data } from './data';
	import { error } from '@sveltejs/kit';

	export let data: Data;
	export let searchTerm: string = '';
	export let searchMessage: string = '';
	export let searchResults: any = undefined;

	let modelName = data.modelName;

	if (typeof data.modelMetadata === 'string') {
		throw error(500, `Model metadata couldn't be loaded. Error: ${data.modelMetadata}`);
	}
	let hasN2GSearch: boolean = data.modelMetadata.availableServices.includes('neuron2graph-search');

	async function n2gSearch() {
		console.log(searchTerm);

		const searchResult = await search(modelName, searchTerm, (message) => {
			searchMessage = message;
		});
		if (typeof searchResult === 'string') {
			searchMessage = `Search failed: ${searchResult}`;
		} else {
			searchResults = searchResult;
		}
	}
</script>

<h1>DeepDecipher model page</h1>
<h2>{modelName}</h2>
{#if hasN2GSearch}
	<div id="search-wrapper">
		<p>
			By searching for a token below, you'll receive a list of neurons that activate to these
			tokens. Be aware that most tokens start with
			<span class="code">" "</span> (e.g.
			<span class="code">"Transformers"</span> would be
			<span class="code">" Transformers"</span>) however, this searches over a token database that
			is trimmed and lowercase (i.e.
			<span class="code">" Transformers"</span> becomes
			<span class="code">"transformers"</span>).
		</p>
		<form on:submit|preventDefault={n2gSearch}>
			<input type="text" value={searchTerm} placeholder="Search..." />
			<button>Search</button>
		</form>
		<div id="search-message">{searchMessage}</div>
		{#if searchResults !== undefined}
			<SearchResults baseUrlExtUi={VIZ_EXT} {modelName} {searchResults} />
		{/if}
	</div>
{:else}
	<div class="not-available">Neuron to Graph search is not available for this model.</div>
{/if}

<style>
	#search-message {
		margin: 0.5em 0;
		font-size: 0.8em;
		color: rgba(0, 0, 0, 0.5);
	}
</style>
