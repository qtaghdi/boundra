import type { LoginRequest } from '../shared/public';

export function login(input: LoginRequest) {
  return { ok: true, userId: `user:${input.email}` };
}
