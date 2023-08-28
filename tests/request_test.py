import time
import requests
from concurrent.futures import ThreadPoolExecutor


# Function to measure network speed
def measure_network_speed(url):
    start_time = time.time()
    response = requests.get(url, stream=True)
    total_length = 0

    for chunk in response.iter_content(chunk_size=1024):
        if chunk:
            total_length += len(chunk)

    end_time = time.time()
    elapsed_time = end_time - start_time
    speed = total_length / elapsed_time  # Bytes per second

    return speed


# Function to fetch data from API
def fetch_data(api_url):
    try:
        start_time = time.time()
        response = requests.get(api_url)
        end_time = time.time()

        if response.status_code == 200:
            data_size = len(response.content)
            elapsed_time = end_time - start_time
            return data_size, elapsed_time
        else:
            return 0, 0
    except Exception as e:
        print(f"An error occurred: {e}")
        return 0, 0


def test_api_throughput(
    start_id,
    end_id,
    base_url="https://deepdecipher.org/api/solu-6l-pile/all/1/",
    runs=3,
):
    avg_data_per_second = 0
    avg_requests_per_second = 0

    # Measure network speed
    network_speed = measure_network_speed(
        "https://deepdecipher.org/api/solu-6l-pile/all/1/0"
    )  # Replace with a URL to a large file

    for _ in range(runs):
        total_data_size = 0
        total_time = 0
        total_requests = 0

        with ThreadPoolExecutor(max_workers=100) as executor:
            futures = []
            for i in range(start_id, end_id + 1):
                api_url = f"{base_url}{i}"
                future = executor.submit(fetch_data, api_url)
                futures.append(future)

            for future in futures:
                data_size, elapsed_time = future.result()
                total_data_size += data_size
                total_time += elapsed_time
                total_requests += 1

        data_per_second = total_data_size / total_time
        requests_per_second = total_requests / total_time

        # Adjust API performance based on network speed
        adjusted_data_per_second = data_per_second / network_speed

        avg_data_per_second += adjusted_data_per_second
        avg_requests_per_second += requests_per_second

    avg_data_per_second /= runs
    avg_requests_per_second /= runs

    print(
        f"Interval: {start_id} - {end_id}. Total Requests: {total_requests} times {runs} runs."
    )
    print(f"Data per Second: {data_per_second:.2f} bytes/s")
    print(f"Requests per Second: {requests_per_second:.2f}")
    print(f"Average Adjusted Data per Second: {avg_data_per_second:.2f}")
    print(f"Average Requests per Second: {avg_requests_per_second:.2f}")
    print(f"---------------------")


if __name__ == "__main__":
    for i, j in [(1, 100), (101, 200), (201, 300), (301, 400), (401, 500)]:
        test_api_throughput(i, j)
