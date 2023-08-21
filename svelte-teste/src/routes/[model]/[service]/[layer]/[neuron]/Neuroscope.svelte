<script lang="ts">
	import { error } from '@sveltejs/kit';
	import { BASE_VIZ_API } from '../../../../../base';
	import TokenViz from './TokenViz.svelte';
	import type { Text } from './neuroscope';
	import * as d3 from 'd3-scale';

	export let texts: Text[];

	function tokens(
		text: Text,
		truncate: boolean
	): { token: string; activation: number; color: string }[] {
		const absActivations = text.activations.map(Math.abs);

		const maxActivationIndex = absActivations.indexOf(Math.max(...absActivations));

		// Scale for coloring the tokens based on activations
		const colorScale = d3
			.scaleLinear()
			.domain([Math.min(...absActivations), Math.max(...absActivations)])
			.range(['#EFEEFF', '#761C6D', '#CC4346', '#F99006', '#F9FC9C'] as any);

		// Determine the start and end of the slice based on the location of the max activating token.
		const startIndex = Math.max(0, maxActivationIndex - 50);
		const endIndex = Math.min(text.tokens.length, maxActivationIndex + 4 + 1); // "+1" because slice end index is exclusive

		const tokens = truncate ? text.tokens.slice(startIndex, endIndex) : text.tokens;
		const activations = truncate ? absActivations.slice(startIndex, endIndex) : absActivations;

		return tokens.map((token, index) => {
			const activation = activations[index];
			const color: string = colorScale(activation) as any;
			return { token, activation, color };
		});
	}

	function toggle_all_tokens(this: HTMLElement) {
		var content = this.nextElementSibling as HTMLElement | null;
		if (!content) {
			console.error("Couldn't find content element for collapsible button.");
			return;
		}
		if (content.style.maxHeight) {
			content.style.removeProperty('maxHeight');
		} else {
			content.style.maxHeight = content.scrollHeight + 'px';
		}
	}
</script>

<div>
	{#each texts as text, index}
		<h2 class="text-title">
			Text {index}<span class="meta-info"
				>{text.min_activation} to {text.max_activation} activation within the range {text.min_range}
				to {text.max_range}. Data index {text.data_index}. Max activating token located at index {text.max_activating_token_index}
				of the text of length {text.tokens.length}.</span
			>
		</h2>
		<div class="token_string">
			{#each tokens(text, true) as { token, activation, color }}
				<TokenViz {token} {activation} {color} />
			{/each}
		</div>
		<button class="collapsible" on:click={toggle_all_tokens}> 💬 Show all tokens in sample </button>
		<div class="content">
			<div class="token_string">
				{#each tokens(text, false) as { token, activation, color }}
					<TokenViz {token} {activation} {color} />
				{/each}
			</div>
		</div>
	{/each}
</div>

<style>
	.meta-info {
		font-size: 0.8em;
		color: rgba(0, 0, 0, 0.5);
		margin-left: 1em;
		white-space: pre-line;
	}

	.text-title {
		margin: 0;
		font-size: 1.2em;
		padding: 0.25em 0.5em;
		line-height: 1em;
		white-space: pre-wrap;
		border: 1px solid grey;
		border-bottom: none;
		background-color: #f1f1f1;
	}

	.token_string {
		border: 1px solid grey;
		padding: 0.5em;
		background-color: #f1f1f1;
	}

	.collapsible {
		cursor: pointer;
		padding: 0.5em;
		width: 100%;
		border: none;
		text-align: left;
		outline: none;
		border: 1px solid grey;
		border-top: none;
		border-bottom: none;
	}

	.content {
		padding: 0 18px;
		max-height: 0;
		overflow: hidden;
		transition: max-height 0.2s ease-out;
		background-color: #f1f1f1;
		border: 1px solid grey;
	}
</style>
