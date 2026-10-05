import { mount } from 'svelte';
import Fixture from './Fixture.svelte';

Object.assign(window, { codeFixture: mount(Fixture, { target: document.body }) });
