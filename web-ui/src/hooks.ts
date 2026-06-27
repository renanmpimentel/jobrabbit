// Data hooks (React Query) + re-export API helpers.
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { getJSON } from "./api";
import type {
  Answer,
  Application,
  CvReview,
  CvVersion,
  DoctorCheck,
  Feedback,
  Job,
  PendingAction,
  Profile,
  SearchVariant,
  Settings,
  Stats,
} from "./api";

export * from "./api";

export function useInvalidate() {
  const qc = useQueryClient();
  return () => qc.invalidateQueries();
}

const q = <T,>(key: string, path: string) =>
  useQuery<T>({ queryKey: [key], queryFn: () => getJSON<T>(path) });

export const useStats = () => q<Stats>("stats", "/stats");
export const useJobs = () => q<Job[]>("jobs", "/jobs");
export const usePending = () => q<PendingAction[]>("pending", "/pending");
export const useApplications = () => q<Application[]>("applications", "/applications");
export const useFeedback = () => q<Feedback[]>("feedback", "/feedback");
export const useVariants = () => q<SearchVariant[]>("variants", "/variants");
export const useProfile = () => q<Profile>("profile", "/profile");
export const useSettings = () => q<Settings>("settings", "/settings");
export const useAnswers = () => q<Answer[]>("answers", "/answers");
export const useCvReview = () => q<CvReview | null>("cv-review", "/cv-review");
export const useCvVersion = () => q<CvVersion | null>("cv-version", "/cv-version");
export const useDoctor = () => q<DoctorCheck[]>("doctor", "/doctor");
