<script lang="ts">
	import { Tooltip } from '@svelte-plugins/tooltips';
	import NeuronLink from '$lib/NeuronLink.svelte';
	import { VIZ_EXT, formatNumber } from '$lib/base';

	export let similarNeurons: { layer: number; neuron: number; similarity: number }[];
	export let modelName: string;

	const numSimilarNeuronsTableRows = 4;

	function similarNeuronsTableDataFunc(
		similarNeurons: { layer: number; neuron: number; similarity: number }[]
	): { layer: number; neuron: number; similarity: number }[][] {
		const sorted = similarNeurons.sort((a, b) => b.similarity - a.similarity);
		const numCols = Math.ceil(sorted.length / numSimilarNeuronsTableRows);
		let tableData = new Array(numSimilarNeuronsTableRows)
			.fill(null)
			.map(() => new Array(0).fill(null));
		for (let i = 0; i < sorted.length; i++) {
			const row = Math.floor(i / numCols);
			tableData[row].push(sorted[i]);
		}
		return tableData;
	}
	$: similarNeuronsTableData = similarNeuronsTableDataFunc(similarNeurons);
</script>

<div id="similar">
	{#if similarNeurons.length > 0}
		<table>
			<tbody>
				{#each similarNeuronsTableData as row}
					<tr>
						{#each row as { layer, neuron, similarity }}
							<td class="neuron"
								><Tooltip content="Similarity: {formatNumber(similarity, 5)}"
									><NeuronLink {modelName} {layer} {neuron} /></Tooltip
								></td
							>
						{/each}
					</tr>
				{/each}
			</tbody>
		</table>
	{:else}
		<div>No similar neurons exist for this neuron.</div>
	{/if}
</div>

<style>
	#similar {
		text-align: center;
		display: flex;
		flex-wrap: wrap;
		flex-direction: row;
	}

	.neuron {
		padding-right: 1.5em;
	}
</style>
