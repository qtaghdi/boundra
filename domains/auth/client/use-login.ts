import type { LoginRequest } from '../shared/public';
import { login } from '../server/login-service';

export async function useLogin(input: LoginRequest) {
  return login(input);
}
