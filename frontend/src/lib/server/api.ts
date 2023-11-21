import { type Pagination } from "~/types/rust.api";

import { BASE_URL } from "./baseUrl";

export const fetchApi = async <TData>(
  url: `/${string}`,
  options?: RequestInit,
) => {
  const res = await fetch(`${BASE_URL}${url}`, {
    ...options,
    next: {
      revalidate: 0,
      ...options?.next,
    },
  })
    .then((res) => res.json() as Promise<TData>)
    .catch((e) => {
      console.error(e);

      return null;
    });

  return res;
};

export const paginationToQuery = (opts: Pagination) => {
  const params = new URLSearchParams();

  if (opts.page) {
    params.set("pagination.page", String(opts.page));
  }

  if (opts.perPage) {
    params.set("pagination.per_page", String(opts.perPage));
  }

  return params;
};
