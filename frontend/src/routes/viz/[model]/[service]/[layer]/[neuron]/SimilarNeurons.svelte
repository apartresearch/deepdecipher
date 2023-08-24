<script lang="ts">
	import { VIZ_EXT } from '$lib/base';

	export let similarNeurons: any[];
	export let modelName: string;
	export let serviceName: string;
</script>

<div id="similar">
	{#if similarNeurons.length > 0}
		{#each similarNeurons as { layer, neuron, similarity }}
			<div class="similar_neurons">
				<a href="/{VIZ_EXT}/{modelName}/{serviceName}/{layer}/{neuron}"> {layer}:{neuron}</a><span
					data-tooltip="The similarity between the current neuron and the similar neuron."
					>E{similarity}</span
				>,
			</div>
		{/each}
	{:else}
		<div>No similar neurons exist for this neuron.</div>
	{/if}
</div>

<style>
	.similar_neurons {
		max-width: 20em;
		text-decoration: none;
	}

	#similar {
		display: flex;
		flex-wrap: wrap;
		flex-direction: row;
	}
</style>
