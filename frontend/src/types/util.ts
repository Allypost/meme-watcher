import { type Simplify } from "type-fest";

export type Falsy = false | 0 | "" | null | undefined;

export type Assign<T, TAssign> = Simplify<TAssign & Omit<T, keyof TAssign>>;

export type Maybe<T> = T | null | undefined;

export type RecursivePartial<T> = {
  [P in keyof T]?: RecursivePartial<T[P]>;
};

export type RecursiveNonPartial<T> = {
  [P in keyof T]-?: RecursiveNonPartial<T[P]>;
};

export type RecursiveMutable<T> = {
  -readonly [P in keyof T]: RecursiveMutable<T[P]>;
};
