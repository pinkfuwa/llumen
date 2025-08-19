import { fetchEventSource } from '@microsoft/fetch-event-source';

export async function CreateSseInternal() {
	// await fetchEventSource('http://localhost:8001/api/chat/sse', {
	// 	onmessage(ev) {
	// 		console.log('a chunk', ev.data);
	// 	},
	// 	body: JSON.stringify({
	// 		id: 2
	// 	}),
	// 	headers: {
	// 		Authorization:
	// 			'v4.local.TgRIgpLkmLWtKYd5dGG6zaDYW8UE68esm2pRIRF4R4UC73ioyOSkc2a6yAUvn2j9O0NIwkyPhgDyZJUvbJ2RKvVFXqOF1QdWSrB9qvQQSvLkNRGlMQQEMo_f6-RCRyFj5-zutNDNy4WN-HBTkSvJzeB8WoLfO-yCK25NQoBhoTkZ5byxDVdFabuqLCZWuXv9wFBzH0cBqHwkPPN1n709ww',
	// 		'Content-Type': 'application/json'
	// 	},
	// 	method: 'POST'
	// });
}
