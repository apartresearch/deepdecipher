<script lang="ts">
	import SearchResults from './SearchResults.svelte';
	import { search } from '$lib/n2g_search';
	import { VIZ_EXT } from '$lib/base';
	import type { Data } from './data';
	import { error } from '@sveltejs/kit';
	import NeuronChooser from './NeuronChooser.svelte';
	import type { ModelMetadata } from '$lib/modelMetadata';
	import Title from '$lib/Title.svelte';

	export let data: Data;
	let searchTerm: string = '';
	export let searchMessage: string = '';
	export let searchResults: any = undefined;

	$: modelName = data.modelName;
	$: serviceName = data.serviceName;

	if (typeof data.modelMetadata === 'string') {
		throw error(500, `Model metadata couldn't be loaded. Error: ${data.modelMetadata}`);
	}
	$: modelMetadata = data.modelMetadata as ModelMetadata;
	$: hasN2GSearch = modelMetadata.availableServices.includes('neuron2graph-search');

	async function n2gSearch() {
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

<h1><Title /> model page</h1>
<NeuronChooser {modelMetadata} {serviceName} />
{#if hasN2GSearch}
	<h2>Neuron2Graph search</h2>
	<div id="search-wrapper">
		<p>
			By searching for a token below, you'll receive a list of neurons that activate to these
			tokens. Be aware that most tokens start with
			<span class="code">" "</span> (e.g.
			<span class="code">"Transformers"</span> would be
			<span class="code">" Transformers"</span>) however, this searches over a token database that
			is trimmed and lowercase (e.g.
			<span class="code">" Transformers"</span> becomes
			<span class="code">"transformers"</span>).
		</p>
		<form on:submit|preventDefault={n2gSearch}>
			<input type="text" bind:value={searchTerm} placeholder="Search..." />
			<button>Search</button>
		</form>
		<div id="search-message">{searchMessage}</div>
		{#if searchResults !== undefined}
			<SearchResults {modelName} {searchResults} />
		{/if}
	</div>
{:else}
	<div class="not-available">Neuron to Graph search is not available for this model.</div>
{/if}

<style>
	#search-wrapper {
		width: 40%;
		text-align: justify;
	}

	#search-message {
		margin: 0.5em 0;
		font-size: 0.8em;
		color: rgba(0, 0, 0, 0.5);
	}
</style>
