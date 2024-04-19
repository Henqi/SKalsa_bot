import json

import requests

url = "https://avoinna24.fi/api/slot"
params = {
    "filter[ismultibooking]": "false",
    "filter[branch_id]": "2b325906-5b7a-11e9-8370-fa163e3c66dd",
    "filter[group_id]": "a17ccc08-838a-11e9-8fd9-fa163e3c66dd",
    "filter[product_id]": "59305e30-8b49-11e9-800b-fa163e3c66dd",
    "filter[user_id]": "d7c92d04-807b-11e9-b480-fa163e3c66dd",
    "filter[date]": "2024-04-24",
    "filter[start]": "2024-04-24",
    "filter[end]": "2024-04-24",
}
headers = {"X-Subdomain": "arenacenter"}

response = requests.get(url, headers=headers, params=params)
print(response.status_code)
print(json.dumps(response.json(), indent=4))
