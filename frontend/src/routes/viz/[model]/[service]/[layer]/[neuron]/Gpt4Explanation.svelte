<script lang="ts">
	export let gpt4ExplanationData: { explanation: string; score: number };
</script>

<div class="neuronExplainer">
	<div>Explanation: {gpt4ExplanationData.explanation}</div>
	<br />
	<div>Score: {gpt4ExplanationData.score}</div>
</div>
