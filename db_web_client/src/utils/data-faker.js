import { faker } from '@faker-js/faker';
import axios from "axios";

async function fakeData() {
    const [,, table, count = 20] = process.argv;
    
    if(!table) {
        console.error('[!] Missing table name');
        return;
    }

    if(!count) {
        console.info('[i] Setting default fake count to 20');
    }

    switch (table) {
        case 'employees': await fakeEmployees(count); break;
        default: console.error('[!] Invalid table name');
    }
}

async function fakeEmployees(count) {
    console.log(`[i] Faking ${count} employees...`)
    await dbQuery('CREATE TABLE employees id int, first_name varchar, last_name varchar, age int, salary int, department varchar, is_teamleader boolean');

    for(let i = 1; i <= count; i++) {
        await dbQuery(`INSERT INTO employees VALUES (${i}, '${faker.name.firstName()}', '${faker.name.lastName()}', ${faker.random.numeric(2)}, ${faker.random.numeric(5)}, '${faker.name.jobArea()}', ${Math.random() > 0.5 ? 'true' : 'false'})`);
    }
}

export async function dbQuery(query) {
    const response = await axios.post('http://localhost:9000/query', { query });
    return response.data;
}

await fakeData();