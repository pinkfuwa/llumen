import xml.etree.ElementTree as ET
import os
from dotenv import load_dotenv
from google import genai
from google.genai import types

load_dotenv("../backend/.env")

import glob
API_KEY = os.getenv("GEMINI_API_KEY")  # Set your Gemini API key in .env as GEMINI_API_KEY

def generate_keywords(description):
	prompt = f"請根據以下新聞描述，生成適合檢索的**名詞**關鍵字，使用逗號分隔。關鍵字還必須包含這些新聞的類別。關鍵字必須是名詞，最多三個字。產生約30個關鍵字，請依據內容決定要英文還是繁體中文：\n{description}"
	client = genai.Client(api_key=API_KEY)
	model = "gemini-2.5-flash"  # You can change to other Gemini models if needed
	contents = [
		types.Content(
			role="user",
			parts=[types.Part.from_text(text=prompt)],
		),
	]
	generate_content_config = types.GenerateContentConfig()
	keywords = ""
	try:
		for chunk in client.models.generate_content_stream(
			model=model,
			contents=contents,
			config=generate_content_config,
		):
			keywords += chunk.text
	except Exception as e:
		print(f"Gemini API error: {e}")
	# Format keywords: remove spaces, separate with Chinese or English comma
	keywords = keywords.replace('，', ',')  # unify to English comma
	keywords_list = [kw.replace(' ', '') for kw in keywords.split(',') if kw.strip()]
	return ','.join(keywords_list)

def process_xml_file(xml_path):
	tree = ET.parse(xml_path)
	root = tree.getroot()
	channel = root.find("channel")
	for item in channel.findall("item"):
		desc_elem = item.find("description")
		if desc_elem is not None:
			description = desc_elem.text or ""
			keywords = generate_keywords(description)
			print(f"{xml_path}: {keywords}")
			# Remove existing <keyword> if present
			for kw_elem in item.findall("keyword"):
				item.remove(kw_elem)
			# Insert new <keyword> after <description>
			keyword_elem = ET.Element("keyword")
			keyword_elem.text = keywords
			# Find index of <description> to insert after
			children = list(item)
			try:
				idx = children.index(desc_elem)
				item.insert(idx + 1, keyword_elem)
			except ValueError:
				item.append(keyword_elem)
	tree.write(xml_path, encoding="utf-8", xml_declaration=True)

def main():
	xml_files = glob.glob("*.xml")
	for xml_path in xml_files:
		process_xml_file(xml_path)

if __name__ == "__main__":
	main()
