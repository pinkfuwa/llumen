<script lang="ts">
	import { APIProvider, GoogleMap, Marker, InfoWindow } from 'svelte-google-maps-api';
	import { writable, type Readable } from 'svelte/store';
	import { PUBLIC_GOOGLE_MAP_API_KEY } from '$env/static/public';
	import { onDestroy } from 'svelte';
	interface Props {
		raw: string;
		markerList: Record<string, string>[];
	}

	let APIKEY = $state<string>(PUBLIC_GOOGLE_MAP_API_KEY);
	console.log('API Key:', APIKEY);

	let { raw, markerList }: Props = $props();

	let central_latidude = 0,
		central_longtitude = 0;
	let locations: {
		id: number;
		position: { lat: number; lng: number };
		title: string;
    rating: number | undefined;
    address: string;
		ref: google.maps.Marker | undefined;
    
	}[] = [];
	if (markerList.length > 0) {
		let idx = 0;
		for (let marker of markerList) {
			central_latidude += parseFloat(marker.latitude);
			central_longtitude += parseFloat(marker.longtitude);
			locations.push({
				id: idx,
				position: { lat: parseFloat(marker.latitude), lng: parseFloat(marker.longtitude) },
				title: marker.displayname || marker.address || 'Unknown',
        rating: marker.rating ? parseFloat(marker.rating) : undefined,
        address: marker.address || 'No address provided',
        ref: undefined,
			});
			console.log('marker', marker);
			idx++;
		}
		central_latidude /= markerList.length;
		central_longtitude /= markerList.length;
	} else {
		central_latidude = 24.794422;
		central_longtitude = 120.988158;
	}

	const mapOptions = {
		center: { lat: central_latidude, lng: central_longtitude },
		zoom: 13
	};

	const selectedMarkerId = writable<number | null>(null);

	function handleMarkerClick(idx: number) {
		selectedMarkerId.set(idx);
    console.log('Marker clicked:', idx);
	}

	function closeInfoWindow() {
		selectedMarkerId.set(null);
	}
</script>

<div class="h-[450px] w-full border border-gray-300">
	{#if !APIKEY}
		<p class="text-center py-8 text-gray-500">
			Please enter your API key above to load the map.
		</p>
	{:else}
		{#key APIKEY}
			<APIProvider apiKey={APIKEY}>
				<GoogleMap
					id="interactive-map"
					options={mapOptions}
					mapContainerStyle="width: 100%; height: 100%;"
				>
					{#each locations as location (location.id)}
						<Marker
							position={location.position}
							options={{ title: location.title }}
							onLoad={(marker) => {
								locations[location.id].ref = marker;
							}}
							onClick={() => handleMarkerClick(location.id)}
						/>
					{/each}

					{#if $selectedMarkerId !== null}
			{#if $selectedMarkerId !== null}
				{@const selectedLocation = locations.find((l) => l.id === $selectedMarkerId)}
				{#if selectedLocation}
          {#key selectedLocation.id}
            <InfoWindow
              anchor={selectedLocation.ref}
              options={{ pixelOffset: new google.maps.Size(0, -40) }}
              on:closeclick={closeInfoWindow}
            >
              <div class="min-w-[6px] p-0 m-0">
                <strong class="block m-0 p-0 font-semibold">{selectedLocation.title}</strong>
                <p class="m-0 p-0 text-sm">⭐{selectedLocation.rating}</p>
                <p class="m-0 p-0 text-xs">{selectedLocation.address}</p>
              </div>
            </InfoWindow>
          {/key}
				{/if}
			{/if}
					{/if}
				</GoogleMap>
			</APIProvider>
		{/key}
	{/if}
</div>
