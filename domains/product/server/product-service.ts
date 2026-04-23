import { listProducts } from '../client/list';

export function getServerProducts() {
  return listProducts({ email: 'a@b.com', password: 'x' });
}
