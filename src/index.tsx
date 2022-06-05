/* @refresh reload */
import { render } from 'solid-js/web';

import App from './app';

import './main.css';

render(() => <App />, document.getElementById('root') as HTMLElement);
