import type { Translations } from '../../types';
import main from '../de';
import steel from '../steel/de';
import steps from '../steps/de';

// While false the coverage gate only forbids going backwards; once true it requires every key.
export const COMPLETE = false;

const full: Translations = { ...main, ...steel, ...steps };
export default full;
