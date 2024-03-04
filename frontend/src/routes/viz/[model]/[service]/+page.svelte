<script lang="ts">
	import SearchResults from './SearchResults.svelte';
	import { search } from '$lib/n2g_search';
	import { VIZ_EXT } from '$lib/base';
	import type { Data } from './data';
	import { error } from '@sveltejs/kit';
	import NeuronChooser from './NeuronChooser.svelte';
	import type { ModelMetadata } from '$lib/modelMetadata';
	import Nav from '$lib/Nav.svelte';

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

<Nav />
<div class="container">
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
				<input name="search-token" type="text" bind:value={searchTerm} placeholder="Search..." />
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
</div>

<style>
	.container {
		max-width: 1140px;
		margin: 0 auto;
		color: #333;
	}

	#search-wrapper {
		margin-top: 2em;
	}

	#search-message {
		margin: 0.5em 0;
		font-size: 0.8em;
		color: rgba(0, 0, 0, 0.5);
	}

	.code {
		white-space: nowrap;
	}
	.not-available {
		background-color: rgba(255, 0, 0, 0.1);
		border: 1px dashed rgba(255, 0, 0, 0.5);
		color: rgba(255, 0, 0, 0.8);
		border-radius: 0.5em;
		padding: 0.4em 1em;
		display: inline-block;
		margin-top: 1em;
	}

	#search-wrapper form {
		display: flex;
	}

	#search-wrapper input {
		flex: 1;
	}

	#search-wrapper button {
		margin-left: 10px;
	}

	#search-wrapper p {
		margin-bottom: 1em;
	}
</style>
