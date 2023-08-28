import time
import requests
from concurrent.futures import ThreadPoolExecutor


def fetch_data(api_url):
    try:
        response = requests.get(api_url)
        if response.status_code == 200:
            return True
        else:
            return False
    except Exception as e:
        print(f"An error occurred: {e}")
        return False


def test_api_throughput(
    start_id, end_id, base_url="https://deepdecipher.org/api/solu-6l-pile/all/1/"
):
    successful_requests = 0
    total_requests = 0

    start_time = time.time()

    with ThreadPoolExecutor(max_workers=100) as executor:
        futures = []
        for i in range(start_id, end_id + 1):
            api_url = f"{base_url}{i}"
            future = executor.submit(fetch_data, api_url)
            futures.append(future)

        for future in futures:
            total_requests += 1
            if future.result():
                successful_requests += 1

    end_time = time.time()
    elapsed_time = end_time - start_time
    requests_per_second = total_requests / elapsed_time

    print(f"Total Requests: {total_requests}")
    print(f"Successful Requests: {successful_requests}")
    print(f"Elapsed Time: {elapsed_time:.2f} seconds")
    print(f"Requests per Second: {requests_per_second:.2f}")


if __name__ == "__main__":
    for i, j in [(1, 100), (101, 200), (201, 300), (301, 400), (401, 500)]:
        test_api_throughput(i, j)
