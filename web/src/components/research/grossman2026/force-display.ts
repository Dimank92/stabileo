export function forceScale(strain: number) {
  return strain ? 1 / strain : 1;
}

export function forceLegend(strain: number) {
  if (strain < 0) return 'compression positive after division by negative ε · strip compressed left to right, end rows held';
  if (strain > 0) return 'tension positive · strip pulled left to right, end rows held';
  return 'zero imposed strain · end rows held';
}
