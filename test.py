from transformer_lens import HookedTransformer
import neuronav as nrnv

model = HookedTransformer.from_pretrained("solu-1l")
tokenizer = model.tokenizer


tokens = [
    tokenizer.convert_ids_to_tokens([token_id])[0] for token_id in range(len(tokenizer))
]

nrnv.scrape_layer_to_files("data", "solu-1l", tokens, 0, 100)
