import type { Translations } from '../../types';
import main from '../fr';
import steel from '../steel/fr';
import steps from '../steps/fr';

// While false the coverage gate only forbids going backwards; once true it requires every key.
export const COMPLETE = false;

const full: Translations = { ...main, ...steel, ...steps };
export default full;
