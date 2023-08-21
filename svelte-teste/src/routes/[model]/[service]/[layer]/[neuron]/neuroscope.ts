export type Text = {
    data_index: number;
    max_activating_token_index: number;
    max_activation: number;
    min_activation: number;
    max_range: number;
    min_range: number;
    tokens: string[];
    activations: number[];
}