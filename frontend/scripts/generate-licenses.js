#!/usr/bin/env node

import { promisify } from 'util';
import { writeFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { init } from 'license-checker-rseidelsohn';

const collectLicenses = promisify(init);
const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');
const outputFile = join(projectRoot, 'THIRDPARTY.txt');

async function generateLicenses() {
	console.log('Generating frontend third-party licenses...');

	try {
		const licenses = await collectLicenses({
			start: projectRoot,
			production: true,
			excludePrivatePackages: true
		});
		const entries = Object.entries(licenses).sort(([a], [b]) => a.localeCompare(b));

		let output = `# Third-Party Licenses - Frontend

This file contains license information for all third-party dependencies used in the frontend.
Generated from package.json and pnpm-lock.yaml.
--------------------------------------------------------------------------------
`;

		for (const [packageName, info] of entries) {
			output += `${packageName}\n\n`;

			if (info.licenses) {
				output += `License: ${info.licenses}\n`;
			}

			if (info.repository) {
				output += `Repository: ${info.repository}\n`;
			}

			if (info.publisher) {
				output += `Publisher: ${info.publisher}\n`;
			}

			if (info.email) {
				output += `Email: ${info.email}\n`;
			}

			if (info.url) {
				output += `URL: ${info.url}\n`;
			}

			if (info.licenseText) {
				output += 'License Text:\n```\n';
				output += info.licenseText.trim();
				output += '\n```\n\n';
			}

			output +=
				'--------------------------------------------------------------------------------\n';
		}

		writeFileSync(outputFile, output, 'utf8');
		console.log(`✓ Generated ${outputFile}`);
		console.log(`✓ Total dependencies: ${entries.length}`);
	} catch (error) {
		console.error('Error generating licenses:', error);
		process.exit(1);
	}
}

generateLicenses();
