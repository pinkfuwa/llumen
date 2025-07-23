let collapsed = $state(true);

export function toggle() {
	collapsed = !collapsed;
}

export function getCollapsed() {
	return collapsed;
}
