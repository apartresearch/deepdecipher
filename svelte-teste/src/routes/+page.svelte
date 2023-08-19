<script>
	async function models() {
		const response = await fetch('http://localhost:8080/api');
		if (response.ok) {
			const data = await response.json();
			return data['models'];
		} else {
			const text = await response.text();
			throw new Error(text);
		}
	}
</script>

<h1>DeepDecipher front page</h1>
<p>
	A web page and API that provides interpretability information from many sources on various
	transformer models.
</p>
<table>
	<tr>
		<td style="padding: 0 1em;"
			><a href="https://github.com/apartresearch/deepdecipher">GitHub</a></td
		>
		<td style="padding: 0 1em;"><a href="/doc">API documentation</a></td>
	</tr>
</table>
<table id="model-table">
	<thead>
		<tr>
			<th>Model</th>
			<th>Activation Function</th>
			<th>Dataset</th>
			<th>Layers</th>
			<th>Neurons per Layer</th>
			<th>Total Neurons</th>
			<th>Total Parameters</th>
			<th>Available Services</th>
		</tr>

		{#await models()}
			<tr>
				<td colspan="8">Loading...</td>
			</tr>
		{:then models}
			{#each models as model}
				<tr>
					<td><a href="/{model['name']}/all">{model['name']}</a></td>
					<td>{model['activation_function']}</td>
					<td>{model['dataset']}</td>
					<td>{model['num_layers']}</td>
					<td>{model['layer_size'].toLocaleString('en-US')}</td>
					<td>{model['num_total_neurons'].toLocaleString('en-US')}</td>
					<td>{model['num_total_parameters'].toLocaleString('en-US')}</td>
					<td>{model['available_services']}</td>
				</tr>
			{/each}
		{:catch error}
			<tr>
				<td colspan="8">{error.message}</td>
			</tr>
		{/await}
	</thead>
	<tbody id="model-table" />
</table>
<div id="tooltip" />

<style>
	#model-table,
	#model-table th,
	#model-table td {
		border: 1px solid;
	}
</style>
