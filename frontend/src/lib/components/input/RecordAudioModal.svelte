<script lang="ts">
	import { Mic } from '@lucide/svelte';
	import Modal from '$lib/ui/Modal.svelte';
	import { _ } from 'svelte-i18n';

	interface Props {
		open?: boolean;
		onAddFiles: (files: File[]) => void;
	}

	let { open = $bindable(false), onAddFiles }: Props = $props();
	let mediaRecorder: MediaRecorder | null = null;
	let audioChunks: Blob[] = [];
	let isRecording = $state(false);
	let recordingTime = $state(0);
	let recordingInterval: number | null = null;
	let shouldSaveRecording = false;
	let recordingError = $state('');

	const supportedAudioMimeTypes = [
		'audio/mp4;codecs=mp4a.40.2',
		'audio/mp4',
		'audio/aac',
		'audio/mpeg',
		'audio/ogg;codecs=opus',
		'audio/ogg',
		'audio/wav'
	];

	function getSupportedAudioMimeType(): string {
		for (const type of supportedAudioMimeTypes) {
			if (MediaRecorder.isTypeSupported(type)) {
				return type;
			}
		}

		return '';
	}

	function getAudioFileExtension(mimeType: string): string {
		const rawType = mimeType.split('/')[1]?.split(';')[0] ?? '';

		switch (rawType) {
			case 'mp4':
			case 'aac':
			case 'x-m4a':
			case 'm4a':
				return 'm4a';
			case 'mpeg':
				return 'mp3';
			case 'ogg':
				return 'ogg';
			case 'wav':
				return 'wav';
			default:
				return rawType || 'm4a';
		}
	}

	function formatTime(seconds: number): string {
		const mins = Math.floor(seconds / 60);
		const secs = seconds % 60;
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	}

	function clearRecordingTimer() {
		if (recordingInterval) {
			clearInterval(recordingInterval);
			recordingInterval = null;
		}
	}

	function resetRecordingState() {
		isRecording = false;
		recordingTime = 0;
		audioChunks = [];
		shouldSaveRecording = false;
		clearRecordingTimer();
	}

	async function startRecording() {
		try {
			recordingError = '';
			shouldSaveRecording = false;
			const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
			const mimeType = getSupportedAudioMimeType();
			if (!mimeType) {
				stream.getTracks().forEach((track) => track.stop());
				recordingError =
					'This browser only supports audio recording formats llumen cannot upload yet. Please use a browser that supports audio/mp4, audio/aac, audio/mpeg, audio/ogg, or audio/wav.';
				return;
			}

			mediaRecorder = new MediaRecorder(stream, { mimeType });
			audioChunks = [];

			mediaRecorder.ondataavailable = (event) => {
				audioChunks.push(event.data);
			};

			mediaRecorder.onstop = () => {
				if (!shouldSaveRecording) {
					stream.getTracks().forEach((track) => track.stop());
					clearRecordingTimer();
					return;
				}

				const audioBlob = new Blob(audioChunks, { type: mimeType });
				const ext = getAudioFileExtension(mimeType);
				const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
				const audioFile = new File([audioBlob], `recording-${timestamp}.${ext}`, {
					type: mimeType
				});
				onAddFiles([audioFile]);

				stream.getTracks().forEach((track) => track.stop());
				clearRecordingTimer();
			};

			mediaRecorder.start();
			isRecording = true;
			recordingTime = 0;
			recordingInterval = setInterval(() => {
				recordingTime += 1;
			}, 1000) as unknown as number;
		} catch (error) {
			console.error('Failed to start recording:', error);
			recordingError = 'Failed to start audio recording.';
		}
	}

	function stopRecording() {
		if (mediaRecorder && isRecording) {
			shouldSaveRecording = true;
			mediaRecorder.stop();
			open = false;
		}
		isRecording = false;
		recordingTime = 0;
		clearRecordingTimer();
	}

	function cancelRecording() {
		if (mediaRecorder && isRecording) {
			shouldSaveRecording = false;
			mediaRecorder.stop();
		}
		resetRecordingState();
		open = false;
	}
</script>

<Modal bind:open title={$_('chat.record_audio_dialog.title')} onClose={cancelRecording}>
	{#snippet children()}
		<div class="space-y-4">
			<p>
				{$_('chat.record_audio_dialog.description')}
			</p>
			{#if recordingError}
				<p class="border-warning/40 bg-warning/10 text-warning rounded-md border px-3 py-2 text-sm">
					{recordingError}
				</p>
			{/if}
			{#if isRecording}
				<div class="flex flex-col items-center gap-4 py-6">
					<div
						class="flex size-16 animate-pulse items-center justify-center rounded-full bg-red-500"
					>
						<Mic class="size-8 text-white" />
					</div>
					<div class="font-mono text-2xl font-semibold">
						{formatTime(recordingTime)}
					</div>
					<p class="text-sm text-text/60">
						{$_('chat.record_audio_dialog.recording')}
					</p>
				</div>
			{:else}
				<div class="flex flex-col items-center gap-4 py-6">
					<button
						onclick={startRecording}
						class="flex size-16 items-center justify-center rounded-full border-2 border-outline bg-primary transition-colors hover:bg-primary/80"
					>
						<Mic class="size-8" />
					</button>
					<p class="text-sm text-text/60">
						{$_('chat.record_audio_dialog.click_to_start')}
					</p>
				</div>
			{/if}
		</div>
	{/snippet}
	{#snippet footer()}
		<button
			onclick={cancelRecording}
			class="rounded-md border border-outline bg-transparent px-4 py-2 transition-colors hover:bg-primary focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden"
		>
			{$_('chat.record_audio_dialog.cancel')}
		</button>
		{#if isRecording}
			<button
				onclick={stopRecording}
				class="rounded-md border border-outline bg-primary px-4 py-2 transition-colors hover:bg-primary/80 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden"
			>
				{$_('chat.record_audio_dialog.stop')}
			</button>
		{/if}
	{/snippet}
</Modal>
