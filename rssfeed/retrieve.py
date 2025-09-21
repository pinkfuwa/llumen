import requests as rq

xml_source = [
    "https://news.ltn.com.tw/rss/world.xml",
    "https://news.ltn.com.tw/rss/sports.xml",
    "https://www.digitimes.com.tw/tech/rss/xml/xmlrss_10_60.xml",
    "https://media.rss.com/amdtechtalk/feed.xml",
]

def fetch_and_save_rss(xml_url, save_path):
    response = rq.get(xml_url)
    if response.status_code == 200:
        with open(save_path, "wb") as f:
            f.write(response.content)
        print(f"Saved RSS feed from {xml_url} to {save_path}")
    else:
        print(f"Failed to fetch RSS feed from {xml_url}. Status code: {response.status_code}")

def main():
    # Fetch and save each RSS feed, with name from source
    for url in xml_source:
        filename = url.split("/")[-1]
        fetch_and_save_rss(url, filename)

if __name__ == "__main__":
    main()