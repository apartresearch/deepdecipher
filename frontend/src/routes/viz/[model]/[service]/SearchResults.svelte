<script lang="ts">
	export let baseUrlExtUi: string;
	export let modelName: string;
	export let searchResults: any;
</script>

<div class="results">
	{#if searchResults.length == 0}
		<div>No results found</div>
	{:else}
		{#each searchResults as { layer, neuron }}
			<a class="result" href="/{baseUrlExtUi}/{modelName}/all/{layer}/{neuron}" target="_blank"
				>{layer}:{neuron} ↗</a
			>
		{/each}
	{/if}
</div>

<style>
	.results {
		display: flex;
		flex-wrap: wrap;
		flex-direction: row;
		max-width: 40em;
	}

	.result {
		min-width: 9em;
		padding: 0.5em 0.2em;
		text-decoration: none;
		color: black;
	}

	.result:hover {
		color: rgba(0, 0, 0, 0.7);
	}
</style>
