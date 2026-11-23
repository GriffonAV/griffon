export type CleanerSelectionValue = {
  paths: string[];
  dry_run: boolean;
  file_types: string[];
};

export function buildCleanerSelectionValue(
  paths: string[],
  dryRun: boolean,
  fileTypes: string[],
): CleanerSelectionValue {
  return {
    paths: [...paths],
    dry_run: dryRun,
    file_types: [...fileTypes],
  };
}

export function toggleCleanerSelectionValue(
  candidatePath: string,
  selectedPaths: string[],
  dryRun: boolean,
  fileTypes: string[],
): CleanerSelectionValue {
  const paths = selectedPaths.includes(candidatePath)
    ? selectedPaths.filter((path) => path !== candidatePath)
    : [...selectedPaths, candidatePath];

  return buildCleanerSelectionValue(paths, dryRun, fileTypes);
}
