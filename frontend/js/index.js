const modelRow = (model) => {
    return `<tr>
        <td><a href="http://localhost:8080/viz/${model.name}/all">${model.name}</a></td>
        <td>${model.activation_function}</td>
        <td>${model.dataset}</td>
        <td>${model.num_layers}</td>
        <td>${model.layer_size}</td>
        <td>${model.num_total_neurons}</td>
        <td>${model.num_total_parameters}</td>
        <td>${model.available_services.filter((service) => service != 'metadata').join(", ")}</td>
    </tr>`
}

fetch(
    `${baseUrl}/${baseExtApi}`
).then(response => response.json()).then(data => {
    const models = data.models;
    const table = document.getElementById('model-table');
    for (model of models) {
        table.innerHTML += modelRow(model);
    }
})