#!/usr/bin/env node

import { exec } from 'child_process';
import { promisify } from 'util';
import { writeFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const execAsync = promisify(exec);
const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');
const outputFile = join(projectRoot, 'THIRDPARTY.txt');

async function generateLicenses() {
	console.log('Generating frontend third-party licenses...');

	try {
		// Run license-checker with JSON output
		const { stdout } = await execAsync(
			'pnpm exec license-checker --json --production --excludePrivatePackages',
			{ cwd: projectRoot, maxBuffer: 10 * 1024 * 1024 }
		);

		const licenses = JSON.parse(stdout);
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

			if (info.licenseFile) {
				output += `License File: ${info.licenseFile}\n`;
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
