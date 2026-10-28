import type { Translations } from '../../types';
import main from '../tr';
import steel from '../steel/tr';
import steps from '../steps/tr';

// While false the coverage gate only forbids going backwards; once true it requires every key.
export const COMPLETE = true;

const full: Translations = { ...main, ...steel, ...steps };
export default full;
