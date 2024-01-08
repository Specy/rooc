import type { Project } from '$src/stores/projectStore';
import Dexie from 'dexie';
import type { Table } from 'dexie';

export type SerializedProject = Project

function createUuid(): string {
    const bytes = crypto.getRandomValues(new Uint32Array(4))
    return [...bytes].map(b => b.toString(16)).join("")
}


class RoocDb extends Dexie{
    projects: Table<SerializedProject, string>

    public constructor(){
        super("rooc_db")
        this.version(1).stores({
            projects: "id"
        })
    }

    public async saveProject(project: Project): Promise<Project>{
        const id = createUuid()
        project.id = id
        await this.projects.put(project)
        return project
    }

    public async loadProjects(): Promise<Project[]>{
        return await this.projects.toArray()
    }
    public async deleteProject(id: string){
        await this.projects.delete(id)
    }
    public async getProject(id: string): Promise<Project>{
        return await this.projects.get(id)
    }
    public async updateProject(id: string, project: Project): Promise<Project>{
        project.updatedAt = new Date().getTime()
        await this.projects.update(id, project)
        project.id = id
        return project
    }

}

export const db = new RoocDb()