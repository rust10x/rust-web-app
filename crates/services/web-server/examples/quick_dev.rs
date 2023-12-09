#![allow(unused)] // For example code.

use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
	let hc = httpc_test::new_client("http://localhost:8080")?;

	// hc.do_get("/index.html").await?.print().await?;

	let req_login = hc.do_post(
		"/api/login",
		json!({
			"username": "demo1",
			"pwd": "welcome"
		}),
	);
	req_login.await?.print().await?;

	let req_create_project = hc.do_post(
		"/api/rpc",
		json!({
			"id": 1,
			"method": "create_project",
			"params": {
				"data": {
					"name": "project AAA"
				}
			}
		}),
	);
	let result = req_create_project.await?;
	result.print().await?;
	let project_id = result.json_value::<i64>("/result/id")?;

	let mut task_ids: Vec<i64> = Vec::new();
	for i in 1..=5 {
		let req_create_task = hc.do_post(
			"/api/rpc",
			json!({
				"id": 1,
				"method": "create_task",
				"params": {
					"data": {
						"project_id": project_id,
						"title": format!("task AAA {i}")
					}
				}
			}),
		);
		let result = req_create_task.await?;
		task_ids.push(result.json_value::<i64>("/result/id")?);
	}

	let req_update_task = hc.do_post(
		"/api/rpc",
		json!({
			"id": 1,
			"method": "update_task",
			"params": {
				"id": task_ids[0], // The first task created.
				"data": {
					"title": "task BB"
				}
			}
		}),
	);
	req_update_task.await?.print().await?;

	let req_delete_task = hc.do_post(
		"/api/rpc",
		json!({
			"id": 1,
			"method": "delete_task",
			"params": {
				"id": task_ids[1] // The second task created.
			}
		}),
	);
	req_delete_task.await?.print().await?;

	let req_list_all_tasks = hc.do_post(
		"/api/rpc",
		json!({
			"id": 1,
			"method": "list_tasks",
			"params": {
				"filters": {
					"project_id": project_id
				},
				"list_options": {
					"order_bys": "!title"
				}
			}
		}),
	);
	req_list_all_tasks.await?.print().await?;

	let req_list_b_tasks = hc.do_post(
		"/api/rpc",
		json!({
			"id": 1,
			"method": "list_tasks",
			"params": {
				"filters": [
				{
					"project_id": project_id,
					"title": {"$contains": "BB"},
				},
				// Shows how to use other $in
				{

					"project_id": { "$in": [project_id] },
					"title": {"$in": ["task AAA 3", "task AAA 4"]}
				},
				// This won't match any projects, so, won't return anything.
				{
					"project_id": { "$in": [ 123, 124]},
					"title": {"$in": ["task AAA 2", "task AAA 5"]}
				}
				]
			}
		}),
	);
	req_list_b_tasks.await?.print().await?;

	let req_logoff = hc.do_post(
		"/api/logoff",
		json!({
			"logoff": true
		}),
	);
	// req_logoff.await?.print().await?;

	Ok(())
}
