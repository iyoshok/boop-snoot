/* @refresh reload */
import { render } from 'solid-js/web';
import './notifications';

import App from './app';

import './main.css';
import './fonts.css';

render(() => <App />, document.getElementById('root') as HTMLElement);
