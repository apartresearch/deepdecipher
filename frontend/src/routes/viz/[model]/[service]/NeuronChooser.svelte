<script lang="ts">
	import { goto } from '$app/navigation';
	import { VIZ_EXT } from '$lib/base';
	import type { ModelMetadata } from '$lib/modelMetadata';
	import { get } from 'svelte/store';
	import {
		navLayerIndexStore,
		navLayerIndices,
		navNeuronIndexStore,
		navNeuronIndices
	} from './stores';

	export let modelMetadata: ModelMetadata;
	export let serviceName: string;

	$: ({ name: modelName, numLayers, layerSize } = modelMetadata);

	let navLayerIndex = 0;
	let navNeuronIndex = 0;
	$: navLayerIndex = get(navLayerIndices)[modelName] ?? 0;
	$: navNeuronIndex = get(navNeuronIndices)[modelName] ?? 0;

	function updateIndices(layerIndex: number, neuronIndex: number) {
		navLayerIndices.update((indices) => ({ ...indices, [modelName]: layerIndex }));
		navNeuronIndices.update((indices) => ({ ...indices, [modelName]: neuronIndex }));

		navLayerIndexStore.set(layerIndex);
		navNeuronIndexStore.set(neuronIndex);
	}

	$: updateIndices(navLayerIndex, navNeuronIndex);

	let navigating: boolean = false;

	function goToNeuron() {
		navigating = true;
		const url = `/${VIZ_EXT}/${modelName}/${serviceName}/${navLayerIndex}/${navNeuronIndex}`;
		goto(url);
	}

	function goToRandom() {
		navigating = true;
		const layerIndex = Math.floor(Math.random() * numLayers);
		const neuronIndex = Math.floor(Math.random() * layerSize);
		const url = `/${VIZ_EXT}/${modelName}/${serviceName}/${layerIndex}/${neuronIndex}`;
		goto(url);
	}
</script>

<h1>
	{modelName}
</h1>
<p>Choose a neuron to visualize</p>
<form on:submit|preventDefault={goToNeuron} class="model-form">
	<div class="form-row">
		<label for="layer-index">Layer</label>
		<input
			id="layer-index"
			name="layer-index"
			type="number"
			bind:value={navLayerIndex}
			min="0"
			max={numLayers - 1}
			placeholder="Layer index..."
		/>
	</div>
	<div class="form-row">
		<label for="neuron-index">Neuron</label>
		<input
			id="neuron-index"
			name="neuron-index"
			type="number"
			bind:value={navNeuronIndex}
			min="0"
			max={layerSize - 1}
			placeholder="Neuron index..."
		/>
	</div>
	<div class="action-buttons">
		<button type="submit">Go!</button>
		<button on:click|preventDefault={goToRandom}>Random</button>
		{#if navigating}<span class="loading-text">Loading...</span>{/if}
	</div>
</form>

<style>
	h1 {
		margin-bottom: 0;
		font-size: 24px;
	}

	p {
		margin-top: 0;
		margin-bottom: 0;
	}

	.model-form {
		width: 100%;
		display: flex;
		flex-direction: row;
		background: #fff;
		border-radius: 8px;
		gap: 10px;
	}

	.form-row {
		display: flex;
		flex-direction: column;
		margin-bottom: 10px;
		padding: 0;
	}

	label {
		flex-basis: 20%;
		margin-right: 10px;
		font-size: 12px;
		margin-bottom: -8px;
		margin-left: 4px;
		background-color: #fff;
		color: #3337;
		z-index: 1;
		padding: 0 4px;
	}

	input {
		flex-grow: 1;
		min-width: 50px;
		padding: 8px;
		border: 1px solid #ccc;
		border-radius: 4px;
	}

	.action-buttons {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		padding-bottom: 4px;
		gap: 10px;
	}

	.action-buttons button {
		padding: 8px 16px;
		background-color: #007bff;
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		transition: background-color 0.3s ease;
	}

	.action-buttons button:hover {
		background-color: #0056b3;
	}

	.loading-text {
		margin-left: 10px;
		color: #007bff;
	}

	/* Responsive adjustments */
	@media (max-width: 768px) {
		.model-form {
			width: 90%;
		}
	}
</style>
