<script lang="ts">
  import {
    ArrowsDownUpIcon,
    CaretDownIcon,
    StackIcon,
    CheckIcon,
    CircleIcon,
    TrayIcon,
    GearSixIcon,
    ListBulletsIcon,
    MagnifyingGlassIcon,
    MinusIcon,
    PencilSimpleIcon,
    PlusIcon,
    SparkleIcon,
    StarFourIcon,
    TrashIcon,
    UploadSimpleIcon,
    DownloadSimpleIcon,
    X,
    SquaresFourIcon,
    FunnelIcon,
    SortAscendingIcon,
    SortDescendingIcon, 
    DotsThreeVerticalIcon, 
    StarIcon, 
    ArchiveIcon,
    WarningDiamondIcon
  } from "phosphor-svelte";
  import type { IconWeight } from "phosphor-svelte";

  export let name: "grid" | "list" | "box" | "deck" | "settings" | "search" | "plus" | "trash" | "x" | "upload" | "download" | "chevronDown" | "filter" | "arrowUpDown" | "sparkles" | "pencil" | "minus" | "check" | "circle" | "astroid" | "sort-asc" | "sort-desc" | "dots-three-vertical" | "star" | "archive" | "warning" = "grid";
  export let size = 16;
  export let weight: IconWeight = "regular";
  export let color = "currentColor";

  const icons = { grid: SquaresFourIcon, list: ListBulletsIcon, box: TrayIcon, deck: StackIcon, settings: GearSixIcon, search: MagnifyingGlassIcon, plus: PlusIcon, pencil: PencilSimpleIcon, trash: TrashIcon, x: X, upload: UploadSimpleIcon, download: DownloadSimpleIcon, chevronDown: CaretDownIcon, filter: FunnelIcon, arrowUpDown: ArrowsDownUpIcon, sparkles: SparkleIcon, minus: MinusIcon, check: CheckIcon, circle: CircleIcon, astroid: StarFourIcon, "sort-asc": SortAscendingIcon, "sort-desc": SortDescendingIcon, "dots-three-vertical": DotsThreeVerticalIcon, star: StarIcon, archive: ArchiveIcon, warning: WarningDiamondIcon };
  $: IconComponent = icons[name];
</script>

<svelte:component this={IconComponent} {...$$restProps} size={size} weight={weight} color={color} aria-hidden="true" />
