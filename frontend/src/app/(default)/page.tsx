import { fetchApi, paginationToQuery } from "~/lib/server/api";
import {
  type PageDataIndex,
  type PageDataIndexItem,
  type Pagination,
} from "@gen-types/backend/api";

import {
  BlurhashImage,
  BlurhashVideo,
  Keybinds,
  PaginationLinks,
} from "./page.client";

const MediaItem = ({ item }: { item: PageDataIndexItem }) => {
  const itemUrl = `/api/file/serve/${item.id}`;
  const itemMimeType = item.fileType;
  const dimensions = item.data.find((x) => x.key === "media-dimensions")
    ?.meta as
    | {
        width: number;
        height: number;
      }
    | undefined;
  const blurHash = item.data.find((x) => x.key === "blurhash")?.value;

  const aspectRatio =
    dimensions?.width && dimensions?.height
      ? `${dimensions.width}/${dimensions.height}`
      : undefined;

  switch (itemMimeType) {
    case "image/jpeg":
    case "image/png":
    case "image/gif":
    case "image/webp": {
      return (
        <BlurhashImage
          alt={item.name}
          blurHash={blurHash}
          className="max-h-full w-full object-contain"
          src={itemUrl}
          style={{
            aspectRatio,
          }}
          {...dimensions}
        />
      );
    }
  }

  if (itemMimeType?.startsWith("video/")) {
    return (
      <BlurhashVideo
        controls
        playsInline
        blurHash={blurHash}
        className="max-h-full w-full cursor-pointer object-contain"
        poster={`${itemUrl}/poster`}
        preload="none"
        style={{
          aspectRatio,
        }}
        {...dimensions}
      >
        <source src={itemUrl} type={itemMimeType} />
      </BlurhashVideo>
    );
  }

  if (itemMimeType?.startsWith("audio/")) {
    return (
      <figure className="flex h-full w-full flex-col items-center justify-center gap-4">
        <audio
          controls
          className="w-full cursor-pointer object-contain"
          preload="none"
        >
          <source src={itemUrl} type={itemMimeType} />
        </audio>
        <figcaption>
          <a
            className="break-all underline hover:no-underline"
            href={itemUrl}
            rel="noopener noreferrer"
            target="_blank"
          >
            {item.name}
          </a>
        </figcaption>
      </figure>
    );
  }

  return (
    <div className="flex h-full w-full flex-col items-center justify-center gap-4">
      <p>
        <a
          className="break-all underline hover:no-underline"
          href={itemUrl}
          rel="noopener noreferrer"
          target="_blank"
        >
          {item.name}
        </a>
      </p>
      <p>{itemMimeType}</p>
    </div>
  );
};

export default async function HomePage({
  searchParams,
}: {
  searchParams: Record<string, string | string[]>;
}) {
  const pagination = {
    page: isNaN(Number(searchParams.page)) ? 1 : Number(searchParams.page),
    perPage: isNaN(Number(searchParams.perPage))
      ? 24
      : Number(searchParams.perPage),
  } satisfies Pagination;
  const queryParams = paginationToQuery(pagination);
  const pageData = await fetchApi<PageDataIndex>(
    `/page-data/index?${queryParams.toString()}`,
  );

  if (!pageData) {
    return null;
  }

  return (
    <>
      <Keybinds
        pagination={pagination}
        totalPages={pageData.pagination.totalPages}
      />

      <PaginationLinks
        className="flex flex-wrap items-center justify-between gap-4 rounded-md bg-black/50 p-2"
        pagination={pagination}
        totalPages={pageData.pagination.totalPages}
      />

      <div className="flex flex-wrap justify-around gap-4 sm:gap-8">
        {pageData.items.map((item) => {
          return (
            <div
              key={item.id}
              className="flex w-full items-center justify-center sm:aspect-[9/16] sm:w-[min(100%,16rem)]"
            >
              <MediaItem item={item} />
            </div>
          );
        })}
      </div>

      <PaginationLinks
        className="flex flex-wrap items-center justify-between gap-4 rounded-md bg-black/50 p-2"
        pagination={pagination}
        totalPages={pageData.pagination.totalPages}
      />
    </>
  );
}
