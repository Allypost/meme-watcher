"use client";

import { decode as decodeBlurhash } from "blurhash";
import Link, { type LinkProps } from "next/link";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import {
  type FC,
  type HTMLProps,
  type PropsWithChildren,
  type ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useIntersectionObserver } from "usehooks-ts";

import { cn } from "~/lib/util/class";

const DisableableLink: FC<
  PropsWithChildren<
    Omit<HTMLProps<HTMLAnchorElement>, "ref"> &
      LinkProps & {
        disabled?: boolean;
        disabledClassName?: string;
      }
  >
> = ({ disabled, disabledClassName, ...props }) => {
  if (disabled) {
    return (
      <Link
        {...props}
        aria-disabled="true"
        className={cn(props.className, disabledClassName)}
        href="#"
        onClick={(e) => {
          e.stopPropagation();
          e.preventDefault();
        }}
      />
    );
  }

  return <Link {...props} />;
};

export const Keybinds: FC<{
  pagination: {
    page: number;
    perPage: number;
  };
  totalPages: number | null | undefined;
}> = ({ pagination, totalPages }) => {
  const router = useRouter();
  const searchParams = useSearchParams();
  const pathname = usePathname();

  const hasPrev = pagination.page > 1;
  const hasNext = Boolean(totalPages && pagination.page < totalPages);

  const paramsFor = useCallback(
    (args: Record<string, string | number | undefined | null>) => {
      const newParams = new URLSearchParams(searchParams);

      for (const [key, value] of Object.entries(args)) {
        if (value === undefined || value === null) {
          newParams.delete(key);
        } else {
          newParams.set(key, String(value));
        }
      }

      return newParams.toString();
    },
    [searchParams],
  );

  useEffect(() => {
    const keybinds = [
      {
        reject: (e) => {
          if (!hasNext) {
            return true;
          }

          if (e.code !== "ArrowRight") {
            return true;
          }
        },

        handler: () => {
          router.push(
            `${pathname}?${paramsFor({
              page: pagination.page + 1,
            })}`,
          );
        },
      },

      {
        reject: (e) => {
          if (!hasPrev) {
            return true;
          }

          if (e.code !== "ArrowLeft") {
            return true;
          }
        },

        handler: () => {
          router.push(
            `${pathname}?${paramsFor({
              page: pagination.page - 1,
            })}`,
          );
        },
      },
    ] as {
      reject: (e: KeyboardEvent) => boolean | null | undefined;
      handler: (e: KeyboardEvent) => unknown;
    }[];

    const handler = (e: KeyboardEvent) => {
      for (const { reject, handler } of keybinds) {
        if (reject(e)) {
          continue;
        }

        e.preventDefault();
        handler(e);
      }
    };

    document.addEventListener("keydown", handler);
    return () => {
      document.removeEventListener("keydown", handler);
    };
  }, [hasNext, hasPrev, pagination.page, paramsFor, pathname, router]);

  return null;
};

export const PaginationLinks = ({
  pagination,
  totalPages,
  ...props
}: HTMLProps<HTMLDivElement> & {
  pagination: {
    page: number;
    perPage: number;
  };
  totalPages: number | null | undefined;
}) => {
  const router = useRouter();
  const searchParams = useSearchParams();

  const hasPrev = pagination.page > 1;
  const hasNext = Boolean(totalPages && pagination.page < totalPages);

  const perPageOptions = useMemo(() => {
    const divisors = [3, 4];
    const ret = [1];

    for (let i = 1; i < 250; i += 1) {
      if (!divisors.every((d) => i % d === 0)) {
        continue;
      }

      ret.push(i);
    }

    return ret;
  }, []);

  const paramsFor = useCallback(
    (args: Record<string, string | number | undefined | null>) => {
      const newParams = new URLSearchParams(searchParams);

      for (const [key, value] of Object.entries(args)) {
        if (value === undefined || value === null) {
          newParams.delete(key);
        } else {
          newParams.set(key, String(value));
        }
      }

      return newParams.toString();
    },
    [searchParams],
  );

  return (
    <div {...props}>
      <DisableableLink
        className="rounded-md bg-purple-900 px-3 py-2 shadow-md hover:bg-purple-800"
        disabled={!hasPrev}
        disabledClassName="opacity-70 !bg-gray-800 cursor-default"
        href={`/?${paramsFor({
          page: pagination.page - 1,
        })}`}
      >
        &larr; Prev
      </DisableableLink>

      <div className="flex justify-center gap-[1ch] max-md:order-last max-md:basis-full">
        <div>
          Page{" "}
          {totalPages ? (
            <select
              className="inline-block cursor-pointer rounded bg-transparent text-white"
              value={pagination.page}
              onChange={(e) => {
                e.preventDefault();

                router.push(
                  `/?${paramsFor({
                    page: e.target.value,
                  })}`,
                );
              }}
            >
              {Array.from({ length: totalPages }, (_, i) => i + 1).map(
                (page) => (
                  <option key={page} className="bg-black" value={page}>
                    {page}
                  </option>
                ),
              )}
            </select>
          ) : (
            pagination.page
          )}
          /{totalPages ?? "?"}
        </div>
        <div>
          (
          <select
            className="inline-block cursor-pointer rounded bg-transparent text-white"
            value={pagination.perPage}
            onChange={(e) => {
              e.preventDefault();

              router.push(
                `/?${paramsFor({
                  perPage: e.target.value,
                })}`,
              );
            }}
          >
            {perPageOptions.map((perPage) => (
              <option key={perPage} className="bg-black" value={perPage}>
                {perPage}
              </option>
            ))}
          </select>{" "}
          per page)
        </div>
      </div>

      <DisableableLink
        className="rounded-md bg-purple-900 px-3 py-2 shadow-md hover:bg-purple-800"
        disabled={!hasNext}
        disabledClassName="opacity-70 !bg-gray-800 cursor-default"
        href={`/?${paramsFor({
          page: pagination.page + 1,
        })}`}
      >
        Next &rarr;
      </DisableableLink>
    </div>
  );
};

const decodeBlurhashToUrl = (hash: string | null | undefined) => {
  if (!hash || typeof document === "undefined") {
    return null;
  }

  const decoded = decodeBlurhash(hash, 32, 32);
  const canvas = document.createElement("canvas");
  canvas.width = 32;
  canvas.height = 32;
  const ctx = canvas.getContext("2d");

  if (!ctx) {
    return null;
  }

  const imageData = ctx.createImageData(32, 32);

  imageData.data.set(decoded);
  ctx.putImageData(imageData, 0, 0);

  return canvas.toDataURL();
};

export type BlurhashResolverProps = {
  blurHash: string | null | undefined;
};

export const BlurhashResolver = ({
  blurHash,
  children,
}: BlurhashResolverProps & {
  children: (blurHashUrl: string | null) => ReactNode | undefined;
}) => {
  const [url, setUrl] = useState<string | null>(null);
  useEffect(() => {
    const newUrl = decodeBlurhashToUrl(blurHash);

    setUrl(newUrl);
  }, [blurHash]);

  return <>{children(url)}</>;
};

export const BlurhashVideo = ({
  blurHash,
  ...props
}: HTMLProps<HTMLVideoElement> & {
  blurHash: string | null | undefined;
}) => {
  return (
    <BlurhashResolver blurHash={blurHash}>
      {(blurHashUrl) => (
        <LazyVideo {...props} placeholderPoster={blurHashUrl} />
      )}
    </BlurhashResolver>
  );
};

export const LazyVideo = ({
  poster,
  placeholderPoster,
  ...props
}: HTMLProps<HTMLVideoElement> & {
  poster?: string | null | undefined;
  placeholderPoster?: string | null | undefined;
}) => {
  const $video = useRef<HTMLVideoElement | null>(null);
  const observer = useIntersectionObserver($video, {
    freezeOnceVisible: true,
    rootMargin: "120px 0px",
  });
  const isIntersecting = observer?.isIntersecting;
  const [isLoaded, setIsLoaded] = useState(false);
  const [actualPoster, setActualPoster] = useState<string | undefined>(
    placeholderPoster ?? undefined,
  );

  useEffect(() => {
    if (!placeholderPoster) {
      return;
    }

    if (actualPoster) {
      return;
    }

    setActualPoster(placeholderPoster);
  }, [actualPoster, placeholderPoster]);

  useEffect(() => {
    if (!isIntersecting) {
      return;
    }

    if (!poster) {
      return;
    }

    if (isLoaded) {
      return;
    }

    const img = new Image();
    img.setAttribute("crossOrigin", "anonymous");
    img.onload = () => {
      setIsLoaded(true);
      setActualPoster(poster);
    };
    img.src = poster;

    () => {
      img.src = "";
      img.onload = null;
    };
  }, [isIntersecting, isLoaded, poster]);

  return <video {...props} ref={$video} poster={actualPoster} />;
};

export const BlurhashImage = ({
  blurHash,
  ...props
}: HTMLProps<HTMLImageElement> & {
  alt: string;
} & BlurhashResolverProps) => {
  return (
    <BlurhashResolver blurHash={blurHash}>
      {(blurHashUrl) => (
        <LazyImage {...props} alt={props.alt} placeholderSrc={blurHashUrl} />
      )}
    </BlurhashResolver>
  );
};

export const LazyImage = ({
  src,
  placeholderSrc,
  ...props
}: HTMLProps<HTMLImageElement> & {
  src?: string | null | undefined;
  placeholderSrc?: string | null | undefined;
}) => {
  const $img = useRef<HTMLImageElement | null>(null);
  const observer = useIntersectionObserver($img, {
    freezeOnceVisible: true,
    rootMargin: "120px 0px",
  });
  const isIntersecting = observer?.isIntersecting;
  const [actualSrc, setActualSrc] = useState<string | undefined>(
    placeholderSrc ?? undefined,
  );
  const isLoaded = actualSrc === src;

  useEffect(() => {
    if (!placeholderSrc) {
      return;
    }

    if (actualSrc) {
      return;
    }

    setActualSrc(placeholderSrc);
  }, [actualSrc, placeholderSrc]);

  useEffect(() => {
    if (!isIntersecting) {
      return;
    }

    if (!src) {
      return;
    }

    if (isLoaded) {
      return;
    }

    const img = new Image();
    img.setAttribute("crossOrigin", "anonymous");
    img.onload = () => {
      setActualSrc(src);
    };
    img.src = src;

    () => {
      img.src = "";
      img.onload = null;
    };
  }, [isIntersecting, isLoaded, src]);

  return (
    <img
      {...props}
      ref={$img}
      alt={props.alt}
      decoding="async"
      loading="lazy"
      src={actualSrc}
      className={cn(props.className, "transition-[filter] duration-500", {
        "blur-md": !isLoaded,
      })}
    />
  );
};
