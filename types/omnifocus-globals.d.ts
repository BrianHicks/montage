declare const app: Application;

declare const console: Console;

declare const document: DatabaseDocument;

declare const inbox: Inbox;

declare const flattenedProjects: ProjectArray;

declare const projects: ProjectArray;

declare const flattenedTags: TagArray;

declare const flattenedTasks: TaskArray;

declare const library: Library;

declare const moveSections = function (
  sections: Array<Project | Folder>,
  position: Folder | Folder.ChildInsertionLocation
): void {};
