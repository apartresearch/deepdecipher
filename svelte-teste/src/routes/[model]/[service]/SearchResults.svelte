<script lang="ts">
	export let baseUrlExtUi: string;
	export let modelName: string;
	export let searchResults: any;
</script>

{#if searchResults.length == 0}
	<div>No results found</div>
{:else}
	{#each searchResults as { layer, neuron }}
		<a class="result" href="/{baseUrlExtUi}/{modelName}/all/{layer}/{neuron}" target="_blank"
			>{layer}:{neuron} ↗</a
		>
	{/each}
{/if}
